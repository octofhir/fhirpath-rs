/**
 * Animated Card Component
 * Enhanced card with smooth animations and micro-interactions
 */

import { Card, CardProps } from "@mantine/core";
import { useState, useRef, useEffect } from "react";
import { useMantineTheme } from "@mantine/core";
import styles from "./animated.module.css";

interface AnimatedCardProps extends CardProps {
  children: React.ReactNode;
  hoverScale?: number;
  hoverRotation?: number;
  animationDelay?: number;
  glowEffect?: boolean;
  rippleEffect?: boolean;
  className?: string;
}

export function AnimatedCard({
  children,
  hoverScale = 1.02,
  hoverRotation = 0,
  animationDelay = 0,
  glowEffect = false,
  rippleEffect = false,
  className = "",
  ...props
}: AnimatedCardProps) {
  const theme = useMantineTheme();
  const [isHovered, setIsHovered] = useState(false);
  const [ripples, setRipples] = useState<Array<{ id: number; x: number; y: number }>>([]);
  const cardRef = useRef<HTMLDivElement>(null);
  const rippleIdRef = useRef(0);

  const handleMouseEnter = () => {
    setIsHovered(true);
  };

  const handleMouseLeave = () => {
    setIsHovered(false);
  };

  const handleClick = (event: React.MouseEvent<HTMLDivElement>) => {
    if (!rippleEffect || !cardRef.current) return;

    const rect = cardRef.current.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;

    const newRipple = {
      id: rippleIdRef.current++,
      x,
      y,
    };

    setRipples(prev => [...prev, newRipple]);

    // Remove ripple after animation
    setTimeout(() => {
      setRipples(prev => prev.filter(ripple => ripple.id !== newRipple.id));
    }, 600);
  };

  const cardStyle = {
    transform: isHovered
      ? `scale(${hoverScale}) rotate(${hoverRotation}deg)`
      : 'scale(1) rotate(0deg)',
    transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
    animationDelay: `${animationDelay}ms`,
    boxShadow: isHovered && glowEffect
      ? `0 10px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04), 0 0 0 1px ${theme.colors.blue[3]}40`
      : undefined,
  };

  useEffect(() => {
    if (cardRef.current) {
      cardRef.current.style.animationDelay = `${animationDelay}ms`;
    }
  }, [animationDelay]);

  return (
    <Card
      ref={cardRef}
      className={`${styles.animatedCard} ${className}`}
      style={cardStyle}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      onClick={handleClick}
      {...props}
    >
      {children}

      {/* Ripple Effect */}
      {rippleEffect && (
        <div className={styles.rippleContainer}>
          {ripples.map(ripple => (
            <div
              key={ripple.id}
              className={styles.ripple}
              style={{
                left: ripple.x,
                top: ripple.y,
              }}
            />
          ))}
        </div>
      )}

      {/* Glow Effect Overlay */}
      {glowEffect && isHovered && (
        <div className={styles.glowOverlay} />
      )}
    </Card>
  );
}

export default AnimatedCard;
